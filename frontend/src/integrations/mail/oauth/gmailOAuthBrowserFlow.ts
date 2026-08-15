const GMAIL_OAUTH_AUTHORIZATION_ORIGIN_V1 = 'https://accounts.google.com'
const GMAIL_OAUTH_AUTHORIZATION_PATH_V1 = '/o/oauth2/v2/auth'
const GMAIL_OAUTH_CALLBACK_CHANNEL_V1 = 'makosh.gmail.oauth.callback.v1'
const GMAIL_OAUTH_CALLBACK_KIND_V1 = 'makosh.gmail.oauth.callback.v1'
const GMAIL_OAUTH_TIMEOUT_MILLIS_V1 = 10 * 60 * 1000
const MAX_GMAIL_OAUTH_CARRIER_BYTES_V1 = 8 * 1024
const MAX_GMAIL_OAUTH_URL_BYTES_V1 = 16 * 1024

export const GMAIL_OAUTH_CALLBACK_PATH_V1 = '/oauth/google/callback'

export type GmailOAuthBrowserResultV1 = {
	returnedState: string
	authorizationCode: string
}

export type GmailOAuthCallbackPageStateV1 = 'not_callback' | 'accepted' | 'rejected'

type GmailOAuthCallbackMessageV1 = {
	kind: typeof GMAIL_OAUTH_CALLBACK_KIND_V1
	state: string
	authorizationCode?: string
	error?: 'provider_rejected' | 'callback_invalid'
}

type GmailOAuthChannelV1 = {
	onmessage: ((event: MessageEvent<unknown>) => void) | null
	postMessage(value: unknown): void
	close(): void
}

type GmailOAuthPopupV1 = {
	close(): void
}

export type GmailOAuthBrowserEnvironmentV1 = {
	location: Pick<Location, 'origin' | 'pathname' | 'search'>
	history: Pick<History, 'replaceState'>
	open(url: string, target: string, features: string): GmailOAuthPopupV1 | null
	createChannel(name: string): GmailOAuthChannelV1
	setTimer(callback: () => void, delayMillis: number): unknown
	clearTimer(handle: unknown): void
}

export function gmailOAuthLoopbackRedirectUriV1(origin: string): string {
	let parsed: URL
	try {
		parsed = new URL(origin)
	} catch {
		throw new Error('gmail_oauth_loopback_origin_required')
	}
	if (
		parsed.origin !== origin
		|| parsed.protocol !== 'http:'
		|| !['127.0.0.1', 'localhost'].includes(parsed.hostname)
		|| !parsed.port
		|| parsed.username
		|| parsed.password
		|| parsed.pathname !== '/'
		|| parsed.search
		|| parsed.hash
	) {
		throw new Error('gmail_oauth_loopback_origin_required')
	}
	return new URL(GMAIL_OAUTH_CALLBACK_PATH_V1, parsed).toString()
}

export async function runGmailOAuthBrowserFlowV1(
	authorizationUrl: string,
	environment: GmailOAuthBrowserEnvironmentV1 = defaultBrowserEnvironmentV1(),
): Promise<GmailOAuthBrowserResultV1> {
	const redirectUri = gmailOAuthLoopbackRedirectUriV1(environment.location.origin)
	const expectedState = validateAuthorizationUrlV1(authorizationUrl, redirectUri)
	const channel = environment.createChannel(callbackChannelNameV1(expectedState))
	let popup: GmailOAuthPopupV1 | null = null
	let timer: unknown
	let settled = false

	return new Promise((resolve, reject) => {
		const finish = (
			result: GmailOAuthBrowserResultV1 | undefined,
			error: Error | undefined,
		): void => {
			if (settled) return
			settled = true
			if (timer !== undefined) environment.clearTimer(timer)
			channel.onmessage = null
			channel.close()
			popup?.close()
			if (result) resolve(result)
			else reject(error ?? new Error('gmail_oauth_callback_invalid'))
		}

		channel.onmessage = (event) => {
			const callback = parseCallbackMessageV1(event.data)
			if (!callback || callback.state !== expectedState) return
			if (callback.error) {
				finish(undefined, new Error(callback.error))
				return
			}
			finish({
				returnedState: callback.state,
				authorizationCode: callback.authorizationCode!,
			}, undefined)
		}
		timer = environment.setTimer(
			() => finish(undefined, new Error('gmail_oauth_callback_timeout')),
			GMAIL_OAUTH_TIMEOUT_MILLIS_V1,
		)
		popup = environment.open(
			authorizationUrl,
			'makosh-gmail-oauth',
			'popup,width=720,height=760,resizable=yes,scrollbars=yes',
		)
		if (!popup) finish(undefined, new Error('gmail_oauth_popup_blocked'))
	})
}

export function completeGmailOAuthBrowserCallbackV1(
	environment: GmailOAuthBrowserEnvironmentV1 = defaultBrowserEnvironmentV1(),
): GmailOAuthCallbackPageStateV1 {
	if (environment.location.pathname !== GMAIL_OAUTH_CALLBACK_PATH_V1) return 'not_callback'
	const callback = callbackMessageFromSearchV1(environment.location.search)
	environment.history.replaceState(null, '', GMAIL_OAUTH_CALLBACK_PATH_V1)
	if (!callback.message.state) return callback.pageState
	const channel = environment.createChannel(callbackChannelNameV1(callback.message.state))
	try {
		channel.postMessage(callback.message)
	} finally {
		channel.close()
	}
	return callback.pageState
}

export function mountGmailOAuthCallbackPageV1(state: Exclude<
	GmailOAuthCallbackPageStateV1,
	'not_callback'
>): void {
	const main = document.createElement('main')
	main.className = 'gmail-oauth-callback'
	const heading = document.createElement('h1')
	const message = document.createElement('p')
	heading.textContent = state === 'accepted'
		? 'Google authorization received'
		: 'Google authorization was not completed'
	message.textContent = state === 'accepted'
		? 'Return to Макошь. This window can now be closed.'
		: 'Return to Макошь and start a new OAuth attempt.'
	main.append(heading, message)
	document.querySelector('#app')?.replaceChildren(main)
}

function validateAuthorizationUrlV1(authorizationUrl: string, redirectUri: string): string {
	if (!boundedAsciiV1(authorizationUrl, MAX_GMAIL_OAUTH_URL_BYTES_V1)) {
		throw new Error('gmail_oauth_authorization_url_invalid')
	}
	let parsed: URL
	try {
		parsed = new URL(authorizationUrl)
	} catch {
		throw new Error('gmail_oauth_authorization_url_invalid')
	}
	if (
		parsed.origin !== GMAIL_OAUTH_AUTHORIZATION_ORIGIN_V1
		|| parsed.pathname !== GMAIL_OAUTH_AUTHORIZATION_PATH_V1
		|| parsed.username
		|| parsed.password
		|| parsed.hash
	) {
		throw new Error('gmail_oauth_authorization_url_invalid')
	}
	if (singleParameterV1(parsed.searchParams, 'redirect_uri') !== redirectUri) {
		throw new Error('gmail_oauth_redirect_uri_mismatch')
	}
	if (singleParameterV1(parsed.searchParams, 'response_type') !== 'code') {
		throw new Error('gmail_oauth_response_type_invalid')
	}
	if (singleParameterV1(parsed.searchParams, 'code_challenge_method') !== 'S256') {
		throw new Error('gmail_oauth_pkce_invalid')
	}
	for (const name of ['client_id', 'code_challenge']) {
		if (!boundedAsciiV1(singleParameterV1(parsed.searchParams, name))) {
			throw new Error('gmail_oauth_authorization_url_invalid')
		}
	}
	const state = singleParameterV1(parsed.searchParams, 'state')
	if (!boundedAsciiV1(state)) throw new Error('gmail_oauth_state_invalid')
	return state
}

function callbackMessageFromSearchV1(search: string): {
	message: GmailOAuthCallbackMessageV1
	pageState: Exclude<GmailOAuthCallbackPageStateV1, 'not_callback'>
} {
	if (!boundedAsciiV1(search, MAX_GMAIL_OAUTH_URL_BYTES_V1)) {
		return invalidCallbackMessageV1('')
	}
	const parameters = new URLSearchParams(search)
	const state = optionalSingleParameterV1(parameters, 'state') ?? ''
	const code = optionalSingleParameterV1(parameters, 'code')
	const providerError = optionalSingleParameterV1(parameters, 'error')
	if (!boundedAsciiV1(state)) return invalidCallbackMessageV1('')
	if (providerError !== undefined && boundedAsciiV1(providerError) && code === undefined) {
		return {
			message: { kind: GMAIL_OAUTH_CALLBACK_KIND_V1, state, error: 'provider_rejected' },
			pageState: 'rejected',
		}
	}
	if (providerError !== undefined || !boundedAsciiV1(code)) {
		return invalidCallbackMessageV1(state)
	}
	return {
		message: {
			kind: GMAIL_OAUTH_CALLBACK_KIND_V1,
			state,
			authorizationCode: code,
		},
		pageState: 'accepted',
	}
}

function invalidCallbackMessageV1(state: string): {
	message: GmailOAuthCallbackMessageV1
	pageState: 'rejected'
} {
	return {
		message: { kind: GMAIL_OAUTH_CALLBACK_KIND_V1, state, error: 'callback_invalid' },
		pageState: 'rejected',
	}
}

function callbackChannelNameV1(state: string): string {
	return `${GMAIL_OAUTH_CALLBACK_CHANNEL_V1}:${state}`
}

function parseCallbackMessageV1(value: unknown): GmailOAuthCallbackMessageV1 | undefined {
	if (!value || typeof value !== 'object' || Array.isArray(value)) return undefined
	const candidate = value as Record<string, unknown>
	if (
		candidate.kind !== GMAIL_OAUTH_CALLBACK_KIND_V1
		|| !boundedAsciiV1(candidate.state)
	) return undefined
	if (candidate.error === 'provider_rejected' || candidate.error === 'callback_invalid') {
		return {
			kind: GMAIL_OAUTH_CALLBACK_KIND_V1,
			state: candidate.state,
			error: candidate.error,
		}
	}
	if (!boundedAsciiV1(candidate.authorizationCode)) return undefined
	return {
		kind: GMAIL_OAUTH_CALLBACK_KIND_V1,
		state: candidate.state,
		authorizationCode: candidate.authorizationCode,
	}
}

function singleParameterV1(parameters: URLSearchParams, name: string): string {
	const values = parameters.getAll(name)
	if (values.length !== 1) throw new Error('gmail_oauth_authorization_url_invalid')
	return values[0]!
}

function optionalSingleParameterV1(
	parameters: URLSearchParams,
	name: string,
): string | undefined {
	const values = parameters.getAll(name)
	return values.length === 1 ? values[0] : undefined
}

function boundedAsciiV1(
	value: unknown,
	maximumBytes = MAX_GMAIL_OAUTH_CARRIER_BYTES_V1,
): value is string {
	return typeof value === 'string'
		&& value.length > 0
		&& value.length <= maximumBytes
		&& /^[\x21-\x7e]+$/.test(value)
}

function defaultBrowserEnvironmentV1(): GmailOAuthBrowserEnvironmentV1 {
	return {
		location: window.location,
		history: window.history,
		open: (url, target, features) => window.open(url, target, features),
		createChannel: (name) => new BroadcastChannel(name),
		setTimer: (callback, delayMillis) => window.setTimeout(callback, delayMillis),
		clearTimer: (handle) => window.clearTimeout(handle as number),
	}
}

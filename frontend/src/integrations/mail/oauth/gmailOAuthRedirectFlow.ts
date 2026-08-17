import { MailGmailOAuthClientV1 } from '../api/mailGmailOAuthClient'
import {
	GMAIL_OAUTH_CALLBACK_PATH_V1,
	gmailOAuthCallbackFromSearchV1,
	gmailOAuthLoopbackRedirectUriV1,
	validateGmailOAuthAuthorizationUrlV1,
} from './gmailOAuthBrowserFlow'

const GMAIL_OAUTH_REDIRECT_STORAGE_KEY_V1 = 'makosh.gmail.oauth.redirect.v1'
const GMAIL_OAUTH_REDIRECT_VERSION_V1 = 1
const GMAIL_OAUTH_REDIRECT_TTL_MILLIS_V1 = 10 * 60 * 1000
const MAX_GMAIL_OAUTH_REDIRECT_RECORD_BYTES_V1 = 12 * 1024
const MAX_GMAIL_OAUTH_CONTINUATION_FIELD_BYTES_V1 = 8 * 1024

export type GmailOAuthSameTabContinuationV1 = {
	operationId: string
	connectionId: string
	setupId: string
}

type GmailOAuthRedirectRecordV1 = GmailOAuthSameTabContinuationV1 & {
	version: typeof GMAIL_OAUTH_REDIRECT_VERSION_V1
	expectedState: string
	createdAtUnixMillis: number
}

type GmailOAuthCompletionPortV1 = Pick<MailGmailOAuthClientV1, 'complete'>

export type GmailOAuthRedirectEnvironmentV1 = {
	location: Pick<Location, 'origin' | 'pathname' | 'search'>
	history: Pick<History, 'replaceState'>
	storage: Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>
	navigate(url: string): void
	now(): number
}

export type GmailOAuthSameTabCallbackStateV1 = 'not_callback' | 'accepted' | 'rejected'

export function redirectGmailOAuthInSameTabV1(
	authorizationUrl: string,
	continuation: GmailOAuthSameTabContinuationV1,
	environment: GmailOAuthRedirectEnvironmentV1 = defaultRedirectEnvironmentV1(),
): void {
	const redirectUri = gmailOAuthLoopbackRedirectUriV1(environment.location.origin)
	const expectedState = validateGmailOAuthAuthorizationUrlV1(authorizationUrl, redirectUri)
	const record: GmailOAuthRedirectRecordV1 = {
		version: GMAIL_OAUTH_REDIRECT_VERSION_V1,
		expectedState,
		createdAtUnixMillis: environment.now(),
		...continuation,
	}
	if (!validRedirectRecordV1(record, environment.now())) {
		throw new Error('gmail_oauth_redirect_continuation_invalid')
	}
	const serialized = JSON.stringify(record)
	if (serialized.length > MAX_GMAIL_OAUTH_REDIRECT_RECORD_BYTES_V1) {
		throw new Error('gmail_oauth_redirect_continuation_invalid')
	}
	environment.storage.setItem(GMAIL_OAUTH_REDIRECT_STORAGE_KEY_V1, serialized)
	environment.navigate(authorizationUrl)
}

export async function completeGmailOAuthSameTabCallbackV1(
	client: GmailOAuthCompletionPortV1 = new MailGmailOAuthClientV1(),
	environment: GmailOAuthRedirectEnvironmentV1 = defaultRedirectEnvironmentV1(),
): Promise<GmailOAuthSameTabCallbackStateV1> {
	if (environment.location.pathname !== GMAIL_OAUTH_CALLBACK_PATH_V1) return 'not_callback'
	const serialized = environment.storage.getItem(GMAIL_OAUTH_REDIRECT_STORAGE_KEY_V1)
	if (serialized === null) return 'not_callback'
	const callbackSearch = environment.location.search

	// Remove provider credentials from the address bar and the one-shot continuation
	// before parsing or awaiting the backend exchange.
	environment.history.replaceState(null, '', GMAIL_OAUTH_CALLBACK_PATH_V1)
	environment.storage.removeItem(GMAIL_OAUTH_REDIRECT_STORAGE_KEY_V1)

	const record = parseRedirectRecordV1(serialized, environment.now())
	const callback = gmailOAuthCallbackFromSearchV1(callbackSearch)
	if (
		!record
		|| callback.pageState !== 'accepted'
		|| callback.message.state !== record.expectedState
		|| !callback.message.authorizationCode
	) return 'rejected'

	try {
		await client.complete({
			operationId: record.operationId,
			connectionId: record.connectionId,
			setupId: record.setupId,
			state: callback.message.state,
			authorizationCode: callback.message.authorizationCode,
		})
		return 'accepted'
	} catch {
		return 'rejected'
	}
}

function parseRedirectRecordV1(
	serialized: string,
	nowUnixMillis: number,
): GmailOAuthRedirectRecordV1 | undefined {
	if (!serialized || serialized.length > MAX_GMAIL_OAUTH_REDIRECT_RECORD_BYTES_V1) return undefined
	try {
		const value: unknown = JSON.parse(serialized)
		return validRedirectRecordV1(value, nowUnixMillis) ? value : undefined
	} catch {
		return undefined
	}
}

function validRedirectRecordV1(
	value: unknown,
	nowUnixMillis: number,
): value is GmailOAuthRedirectRecordV1 {
	if (!value || typeof value !== 'object' || Array.isArray(value)) return false
	const candidate = value as Record<string, unknown>
	return candidate.version === GMAIL_OAUTH_REDIRECT_VERSION_V1
		&& boundedAsciiV1(candidate.expectedState)
		&& boundedAsciiV1(candidate.operationId, 128)
		&& boundedAsciiV1(candidate.connectionId, 128)
		&& boundedAsciiV1(candidate.setupId, 128)
		&& typeof candidate.createdAtUnixMillis === 'number'
		&& Number.isSafeInteger(candidate.createdAtUnixMillis)
		&& candidate.createdAtUnixMillis >= 0
		&& nowUnixMillis >= candidate.createdAtUnixMillis
		&& nowUnixMillis - candidate.createdAtUnixMillis <= GMAIL_OAUTH_REDIRECT_TTL_MILLIS_V1
}

function boundedAsciiV1(
	value: unknown,
	maximumBytes = MAX_GMAIL_OAUTH_CONTINUATION_FIELD_BYTES_V1,
): value is string {
	return typeof value === 'string'
		&& value.length > 0
		&& value.length <= maximumBytes
		&& /^[\x21-\x7e]+$/.test(value)
}

function defaultRedirectEnvironmentV1(): GmailOAuthRedirectEnvironmentV1 {
	return {
		location: window.location,
		history: window.history,
		storage: window.sessionStorage,
		navigate: (url) => window.location.assign(url),
		now: () => Date.now(),
	}
}

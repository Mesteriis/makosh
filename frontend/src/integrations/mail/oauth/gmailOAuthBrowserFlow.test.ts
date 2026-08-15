import { describe, expect, it, vi } from 'vitest'

import {
	GMAIL_OAUTH_CALLBACK_PATH_V1,
	completeGmailOAuthBrowserCallbackV1,
	gmailOAuthLoopbackRedirectUriV1,
	runGmailOAuthBrowserFlowV1,
	type GmailOAuthBrowserEnvironmentV1,
} from './gmailOAuthBrowserFlow'

describe('Gmail OAuth browser flow', () => {
	it('derives one exact loopback callback without accepting ambient paths', () => {
		expect(gmailOAuthLoopbackRedirectUriV1('http://127.0.0.1:5173')).toBe(
			`http://127.0.0.1:5173${GMAIL_OAUTH_CALLBACK_PATH_V1}`,
		)
		expect(() => gmailOAuthLoopbackRedirectUriV1('tauri://localhost')).toThrow(
			'gmail_oauth_loopback_origin_required',
		)
		expect(() => gmailOAuthLoopbackRedirectUriV1('http://example.test')).toThrow(
			'gmail_oauth_loopback_origin_required',
		)
	})

	it('opens only the exact Google authorization endpoint and consumes one matching callback', async () => {
		const channel = new FakeChannel()
		const popup = { close: vi.fn() }
		const environment = browserEnvironment(channel, popup)
		const authorizationUrl = googleAuthorizationUrl('provider-state')
		const resultPromise = runGmailOAuthBrowserFlowV1(authorizationUrl, environment)
		expect(environment.createChannel).toHaveBeenCalledWith(
			'makosh.gmail.oauth.callback.v1:provider-state',
		)

		expect(environment.open).toHaveBeenCalledWith(
			authorizationUrl,
			'makosh-gmail-oauth',
			expect.stringContaining('popup'),
		)
		channel.receive({
			kind: 'makosh.gmail.oauth.callback.v1',
			state: 'provider-state',
			authorizationCode: 'one-use-code',
		})

		await expect(resultPromise).resolves.toEqual({
			returnedState: 'provider-state',
			authorizationCode: 'one-use-code',
		})
		expect(popup.close).toHaveBeenCalledOnce()
		expect(channel.closed).toBe(true)
	})

	it('rejects a redirect mismatch before opening any browser surface', async () => {
		const channel = new FakeChannel()
		const environment = browserEnvironment(channel, { close: vi.fn() })
		const authorizationUrl = new URL(googleAuthorizationUrl('provider-state'))
		authorizationUrl.searchParams.set('redirect_uri', 'http://127.0.0.1:9444/wrong')

		await expect(runGmailOAuthBrowserFlowV1(authorizationUrl.toString(), environment)).rejects
			.toThrow('gmail_oauth_redirect_uri_mismatch')
		expect(environment.open).not.toHaveBeenCalled()
	})

	it('removes provider code from browser history before publishing the callback', () => {
		const channel = new FakeChannel()
		const order: string[] = []
		const environment = browserEnvironment(channel, { close: vi.fn() }, {
			pathname: GMAIL_OAUTH_CALLBACK_PATH_V1,
			search: '?state=provider-state&code=one-use-code',
			replaceState: vi.fn(() => order.push('history')),
		})
		channel.onPost = () => order.push('channel')

		expect(completeGmailOAuthBrowserCallbackV1(environment)).toBe('accepted')
		expect(environment.createChannel).toHaveBeenCalledWith(
			'makosh.gmail.oauth.callback.v1:provider-state',
		)
		expect(order).toEqual(['history', 'channel'])
		expect(environment.history.replaceState).toHaveBeenCalledWith(
			null,
			'',
			GMAIL_OAUTH_CALLBACK_PATH_V1,
		)
		expect(channel.posted).toEqual([{
			kind: 'makosh.gmail.oauth.callback.v1',
			state: 'provider-state',
			authorizationCode: 'one-use-code',
		}])
	})
})

class FakeChannel {
	onmessage: ((event: MessageEvent<unknown>) => void) | null = null
	posted: unknown[] = []
	closed = false
	onPost: (() => void) | undefined

	postMessage(value: unknown): void {
		this.posted.push(value)
		this.onPost?.()
	}

	close(): void {
		this.closed = true
	}

	receive(value: unknown): void {
		this.onmessage?.({ data: value } as MessageEvent<unknown>)
	}
}

function browserEnvironment(
	channel: FakeChannel,
	popup: { close(): void },
	override: {
		pathname?: string
		search?: string
		replaceState?: (data: unknown, unused: string, url?: string | URL | null) => void
	} = {},
): GmailOAuthBrowserEnvironmentV1 & {
	open: ReturnType<typeof vi.fn>
	createChannel: ReturnType<typeof vi.fn>
} {
	return {
		location: {
			origin: 'http://127.0.0.1:5173',
			pathname: override.pathname ?? '/',
			search: override.search ?? '',
		},
		history: {
			replaceState: override.replaceState ?? vi.fn(),
		},
		open: vi.fn(() => popup),
		createChannel: vi.fn(() => channel),
		setTimer: vi.fn(() => 7),
		clearTimer: vi.fn(),
	}
}

function googleAuthorizationUrl(state: string): string {
	const url = new URL('https://accounts.google.com/o/oauth2/v2/auth')
	url.searchParams.set('client_id', 'public-client.apps.googleusercontent.com')
	url.searchParams.set('redirect_uri', `http://127.0.0.1:5173${GMAIL_OAUTH_CALLBACK_PATH_V1}`)
	url.searchParams.set('response_type', 'code')
	url.searchParams.set('state', state)
	url.searchParams.set('code_challenge', 'pkce-challenge')
	url.searchParams.set('code_challenge_method', 'S256')
	return url.toString()
}

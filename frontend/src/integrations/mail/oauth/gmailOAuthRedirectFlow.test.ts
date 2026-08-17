import { describe, expect, it, vi } from 'vitest'

import {
	completeGmailOAuthSameTabCallbackV1,
	redirectGmailOAuthInSameTabV1,
} from './gmailOAuthRedirectFlow'

describe('Gmail OAuth same-tab fallback', () => {
	it('stores only bounded continuation metadata before navigating to Google', () => {
		const storage = memoryStorage()
		const navigate = vi.fn()
		const authorizationUrl = googleAuthorizationUrl('provider-state')

		redirectGmailOAuthInSameTabV1(authorizationUrl, {
			operationId: 'operation-1',
			connectionId: 'gmail-1',
			setupId: 'setup-1',
		}, environment(storage, navigate))

		expect(navigate).toHaveBeenCalledWith(authorizationUrl)
		const persisted = [...storage.values()][0] ?? ''
		expect(persisted).toContain('operation-1')
		expect(persisted).toContain('provider-state')
		expect(persisted).not.toContain('authorizationCode')
		expect(persisted).not.toContain('provider-code')
	})

	it('strips the provider code before completing the backend exchange', async () => {
		const storage = memoryStorage()
		const callbackLocation = {
			origin: 'http://127.0.0.1:5173',
			pathname: '/oauth/google/callback',
			search: '?state=provider-state&code=provider-code',
		}
		const history = { replaceState: vi.fn(() => { callbackLocation.search = '' }) }
		redirectGmailOAuthInSameTabV1(googleAuthorizationUrl('provider-state'), {
			operationId: 'operation-1',
			connectionId: 'gmail-1',
			setupId: 'setup-1',
		}, environment(storage, vi.fn()))
		const complete = vi.fn().mockImplementation(async () => {
			expect(history.replaceState).toHaveBeenCalledWith(
				null,
				'',
				'/oauth/google/callback',
			)
		})

		const result = await completeGmailOAuthSameTabCallbackV1(
			{ complete },
			{
				...environment(storage, vi.fn()),
				location: callbackLocation,
				history,
			},
		)

		expect(result).toBe('accepted')
		expect(complete).toHaveBeenCalledWith({
			operationId: 'operation-1',
			connectionId: 'gmail-1',
			setupId: 'setup-1',
			state: 'provider-state',
			authorizationCode: 'provider-code',
		})
		expect(storage.size).toBe(0)
	})

	it('rejects a callback whose state does not match the pending continuation', async () => {
		const storage = memoryStorage()
		redirectGmailOAuthInSameTabV1(googleAuthorizationUrl('provider-state'), {
			operationId: 'operation-1',
			connectionId: 'gmail-1',
			setupId: 'setup-1',
		}, environment(storage, vi.fn()))
		const complete = vi.fn()

		expect(await completeGmailOAuthSameTabCallbackV1(
			{ complete },
			{
				...environment(storage, vi.fn()),
				location: {
					origin: 'http://127.0.0.1:5173',
					pathname: '/oauth/google/callback',
					search: '?state=wrong-state&code=provider-code',
				},
			},
		)).toBe('rejected')
		expect(complete).not.toHaveBeenCalled()
	})
})

function memoryStorage(): Map<string, string> & {
	getItem(key: string): string | null
	setItem(key: string, value: string): void
	removeItem(key: string): void
} {
	const storage = new Map<string, string>() as ReturnType<typeof memoryStorage>
	storage.getItem = (key) => storage.get(key) ?? null
	storage.setItem = (key, value) => { storage.set(key, value) }
	storage.removeItem = (key) => { storage.delete(key) }
	return storage
}

function environment(storage: ReturnType<typeof memoryStorage>, navigate: (url: string) => void) {
	return {
		location: {
			origin: 'http://127.0.0.1:5173',
			pathname: '/',
			search: '',
		},
		history: { replaceState: vi.fn() },
		storage,
		navigate,
		now: () => 1_000,
	}
}

function googleAuthorizationUrl(state: string): string {
	const url = new URL('https://accounts.google.com/o/oauth2/v2/auth')
	url.searchParams.set('client_id', 'public-client.apps.googleusercontent.com')
	url.searchParams.set('redirect_uri', 'http://127.0.0.1:5173/oauth/google/callback')
	url.searchParams.set('response_type', 'code')
	url.searchParams.set('state', state)
	url.searchParams.set('code_challenge', 'pkce-challenge')
	url.searchParams.set('code_challenge_method', 'S256')
	return url.toString()
}

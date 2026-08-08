import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { readBrowserGatewayCredentialId, storeBrowserGatewayCredentialId } from './browserGatewayCredential'

describe('browser gateway credential storage', () => {
	let entries: Map<string, string>

	beforeEach(() => {
		entries = new Map()
		vi.stubGlobal('localStorage', {
			getItem: (key: string) => entries.get(key) ?? null,
			setItem: (key: string, value: string) => { entries.set(key, value) },
		})
	})

	afterEach(() => vi.unstubAllGlobals())

	it('persists only the public WebAuthn credential identifier', () => {
		storeBrowserGatewayCredentialId('credential-id')

		expect(readBrowserGatewayCredentialId()).toBe('credential-id')
		expect([...entries]).toEqual([['makosh.browser.credential-id.v1', 'credential-id']])
	})

	it('fails closed when browser storage is unavailable', () => {
		vi.stubGlobal('localStorage', { getItem: () => { throw new Error('blocked') } })

		expect(readBrowserGatewayCredentialId()).toBe('')
	})
})

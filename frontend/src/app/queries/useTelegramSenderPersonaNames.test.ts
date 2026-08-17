import { beforeEach, describe, expect, it, vi } from 'vitest'

const client = vi.hoisted(() => ({
	getProfile: vi.fn(),
	listDirectory: vi.fn(),
	listSourceLinks: vi.fn(),
}))

vi.mock('../../platform/connect/personsClient', () => ({
	getPersonsQueryClient: () => client,
}))

import { providerSourceIdentityKey } from '../../shared/identity/providerSourceIdentity'
import { useTelegramSenderPersonaNames } from './useTelegramSenderPersonaNames'

describe('Telegram sender Persona names', () => {
	beforeEach(() => vi.clearAllMocks())

	it('indexes only exact confirmed provider source links by the Persona display name', async () => {
		const id = (value: number) => new Uint8Array(16).fill(value)
		const source = {
			integrationPublicId: id(1),
			accountPublicId: id(2),
			providerSourceContactPublicId: id(3),
		}
		client.listDirectory.mockResolvedValue({
			persons: [{ personId: id(9), displayName: 'Directory Name' }],
			nextAfterPersonId: new Uint8Array(),
		})
		client.getProfile.mockResolvedValue({
			ownerProfile: { displayName: 'Persona Name' },
		})
		client.listSourceLinks.mockResolvedValue({
			sourceLinks: [
				{ source },
				{ source: { ...source, accountPublicId: new Uint8Array([1]) } },
			],
			nextAfterSourceLinkId: new Uint8Array(),
		})

		const index = useTelegramSenderPersonaNames()
		await index.refresh()

		const key = providerSourceIdentityKey(source)
		expect(key).toBeDefined()
		expect(index.names.value.size).toBe(1)
		expect(index.names.value.get(key!)).toBe('Persona Name')
	})
})

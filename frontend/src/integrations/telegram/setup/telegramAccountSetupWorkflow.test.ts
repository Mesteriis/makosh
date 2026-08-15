import { describe, expect, it, vi } from 'vitest'

import { TelegramAccountSetupWorkflowV1 } from './telegramAccountSetupWorkflow'

describe('TelegramAccountSetupWorkflowV1', () => {
	it('provisions both credential purposes before the first runtime apply', async () => {
		const order: string[] = []
		const provision = vi.fn().mockImplementation(async (input) => {
			order.push(input.purposeId)
			return { secretRevision: 1n }
		})
		const apply = vi.fn().mockImplementation(async () => {
			order.push('settings_apply')
			return { settings: { desiredRevision: 2n }, application: {} }
		})
		const lifecycle = vi.fn().mockImplementation(async () => {
			order.push('lifecycle')
			return { accountId: 'personal' }
		})
		const workflow = new TelegramAccountSetupWorkflowV1({
			configuration: { apply },
			vault: { provision },
			lifecycle: { provision: lifecycle },
		} as never)

		await workflow.setup({
			registrationId: 'telegram-registration',
			expectedDesiredRevision: 1n,
			accountId: 'personal',
			displayName: 'Personal',
			apiId: 42n,
			apiHash: new TextEncoder().encode('hash'),
		})

		expect(order).toEqual([
			'telegram_api_hash',
			'telegram_session_store_key',
			'settings_apply',
			'lifecycle',
		])
		expect(lifecycle).toHaveBeenCalledWith(expect.objectContaining({
			credentials: [
				{ purpose: 'telegram_api_hash', revision: 1n },
				{ purpose: 'telegram_session_encryption_key', revision: 1n },
			],
		}))
		expect(provision).toHaveBeenCalledWith(expect.objectContaining({
			configurationInstanceId: 'telegram-registration',
		}))
		expect(apply).toHaveBeenCalledWith(expect.objectContaining({
			configurationInstanceId: 'telegram-registration',
			expectedDesiredRevision: 1n,
		}))
	})

	it('keeps a development API hash in the native custodied sealer', async () => {
		const provision = vi.fn().mockResolvedValue({ secretRevision: 1n })
		const provisionCustodied = vi.fn().mockImplementation(async (_input, seal) => {
			await seal({ hostSessionId: 'native-session' })
			return { secretRevision: 2n }
		})
		const sealApiHash = vi.fn().mockResolvedValue({})
		const lifecycle = vi.fn().mockResolvedValue({ accountId: 'personal' })
		const workflow = new TelegramAccountSetupWorkflowV1({
			configuration: {
				apply: vi.fn().mockResolvedValue({
					settings: { desiredRevision: 2n },
					application: {},
				}),
			},
			vault: { provision, provisionCustodied },
			lifecycle: { provision: lifecycle },
		} as never)

		await workflow.setup({
			registrationId: 'telegram-registration',
			expectedDesiredRevision: 1n,
			accountId: 'personal',
			displayName: 'Personal',
			apiId: 42n,
			apiHashSealer: sealApiHash,
			replaceExistingCredentials: true,
		})

		expect(provisionCustodied).toHaveBeenCalledWith(expect.objectContaining({
			purposeId: 'telegram_api_hash',
			secretRevision: 2n,
		}), sealApiHash)
		expect(sealApiHash).toHaveBeenCalledWith({ hostSessionId: 'native-session' })
		expect(provision).toHaveBeenCalledTimes(1)
		expect(provision).toHaveBeenCalledWith(expect.objectContaining({
			purposeId: 'telegram_session_store_key',
			secretRevision: 2n,
		}))
		expect(lifecycle).toHaveBeenCalledWith(expect.objectContaining({
			credentials: expect.arrayContaining([
				{ purpose: 'telegram_api_hash', revision: 2n },
			]),
		}))
	})
})

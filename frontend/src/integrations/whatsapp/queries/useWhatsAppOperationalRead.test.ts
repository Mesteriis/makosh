import { create } from '@bufbuild/protobuf'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	getWhatsAppOperationalRuntimeStatus,
	listWhatsAppOperationalDialogs,
	listWhatsAppOperationalEvents,
	listWhatsAppOperationalMessages,
	listWhatsAppOperationalParticipants,
	searchWhatsAppOperationalMessages,
} from '../api/whatsAppOperationalReadGateway'
import { openWhatsAppOperationalRealtime } from '../api/whatsAppOperationalRealtime'
import { useWhatsAppOperationalRead } from './useWhatsAppOperationalRead'

vi.mock('../api/whatsAppOperationalReadGateway', () => ({
	getWhatsAppOperationalRuntimeStatus: vi.fn(),
	listWhatsAppOperationalDialogs: vi.fn(),
	listWhatsAppOperationalEvents: vi.fn(),
	listWhatsAppOperationalMessages: vi.fn(),
	listWhatsAppOperationalParticipants: vi.fn(),
	searchWhatsAppOperationalMessages: vi.fn(),
}))

vi.mock('../api/whatsAppOperationalRealtime', () => ({
	openWhatsAppOperationalRealtime: vi.fn(() => ({ close: vi.fn() })),
}))

describe('WhatsApp operational read controller', () => {
	beforeEach(() => {
		vi.clearAllMocks()
		vi.mocked(openWhatsAppOperationalRealtime).mockReturnValue({ close: vi.fn() })
		vi.mocked(getWhatsAppOperationalRuntimeStatus).mockResolvedValue({
			accountId: 'account-1',
			runtimeState: 'connected',
			projectionReady: true,
			latestEventSequence: 4n,
		} as never)
		vi.mocked(listWhatsAppOperationalDialogs).mockResolvedValue({
			item: [{
				providerChatId: 'chat-1',
				title: 'Clean room',
				kind: 'group',
				observedAtUnixSeconds: 1_700_000_000n,
			}],
			nextCursor: 'dialogs-2',
		} as never)
		vi.mocked(listWhatsAppOperationalEvents).mockResolvedValue({
			item: [],
		} as never)
		vi.mocked(listWhatsAppOperationalMessages).mockResolvedValue({
			item: [{
				providerChatId: 'chat-1',
				providerMessageId: 'message-1',
				senderDisplayName: 'Owner',
				text: 'Hello',
				occurredAtUnixSeconds: 1_700_000_000n,
			}],
		} as never)
		vi.mocked(listWhatsAppOperationalParticipants).mockResolvedValue({
			item: [{
				providerChatId: 'chat-1',
				providerIdentityId: 'owner',
				displayName: 'Owner',
				role: 'admin',
				status: 'available',
				observedAtUnixSeconds: 1_700_000_000n,
			}],
		} as never)
		vi.mocked(searchWhatsAppOperationalMessages).mockResolvedValue({
			item: [],
		} as never)
	})

	it('refreshes an admitted account from shared realtime without clearing the visible chat', async () => {
		let onProjectionChanged: ((revision: bigint) => void) | undefined
		vi.mocked(openWhatsAppOperationalRealtime).mockImplementation((_accountId, input) => {
			onProjectionChanged = input.onProjectionChanged
			return { close: vi.fn() }
		})
		const controller = useWhatsAppOperationalRead({
			canQuery: () => true,
			canRealtime: () => true,
			modules: () => [whatsAppModule('whatsapp.operational.query.v1')],
		})
		await controller.reconcile()
		vi.mocked(listWhatsAppOperationalMessages).mockResolvedValue({
			item: [{
				providerChatId: 'chat-1',
				providerMessageId: 'message-2',
				senderDisplayName: 'Owner',
				text: 'Updated',
				occurredAtUnixSeconds: 1_700_000_001n,
			}],
		} as never)

		onProjectionChanged?.(2n)
		expect(controller.model.value).toMatchObject({
			state: 'ready',
			selectedChatId: 'chat-1',
		})
		await vi.waitFor(() => {
			expect(controller.model.value.messages[0]?.text).toBe('Updated')
		})
		controller.stopRealtime()
	})

	it('loads runtime, dialogs, messages, participants and search through exact gateways', async () => {
		const controller = useWhatsAppOperationalRead({
			canQuery: () => true,
			modules: () => [whatsAppModule('whatsapp.operational.query.v1')],
		})

		await controller.reconcile()
		controller.updateSearchQuery(' Hello ')
		await controller.search()

		expect(getWhatsAppOperationalRuntimeStatus).toHaveBeenCalledWith('account-1')
		expect(listWhatsAppOperationalDialogs).toHaveBeenCalledWith({ accountId: 'account-1' })
		expect(listWhatsAppOperationalMessages).toHaveBeenCalledWith({
			accountId: 'account-1',
			providerChatId: 'chat-1',
		})
		expect(listWhatsAppOperationalParticipants).toHaveBeenCalledWith({
			accountId: 'account-1',
			providerChatId: 'chat-1',
		})
		expect(searchWhatsAppOperationalMessages).toHaveBeenCalledWith({
			accountId: 'account-1',
			providerChatId: 'chat-1',
			searchQuery: 'Hello',
		})
		expect(controller.model.value).toMatchObject({
			state: 'ready',
			selectedAccountId: 'account-1',
			selectedChatId: 'chat-1',
			hasMoreDialogs: true,
		})
		expect(controller.model.value.messages[0]?.text).toBe('Hello')
	})

	it('loads the next exact page without duplicating existing dialogs', async () => {
		const controller = useWhatsAppOperationalRead({
			canQuery: () => true,
			modules: () => [whatsAppModule('whatsapp.operational.query.v1')],
		})
		await controller.reconcile()
		vi.mocked(listWhatsAppOperationalDialogs).mockResolvedValueOnce({
			item: [{
				providerChatId: 'chat-1',
				title: 'Duplicate',
				observedAtUnixSeconds: 1_700_000_000n,
			}, {
				providerChatId: 'chat-2',
				title: 'Second',
				observedAtUnixSeconds: 1_700_000_000n,
			}],
		} as never)

		await controller.loadMoreDialogs()

		expect(listWhatsAppOperationalDialogs).toHaveBeenLastCalledWith({
			accountId: 'account-1',
			cursor: 'dialogs-2',
		})
		expect(controller.model.value.dialogs.map(({ id }) => id)).toEqual(['chat-1', 'chat-2'])
	})

	it('keeps the cached account projection visible while a returning account refreshes', async () => {
		const primaryDialogs = deferred<never>()
		let blockPrimaryRefresh = false
		vi.mocked(listWhatsAppOperationalDialogs).mockImplementation(({ accountId }) => {
			if (accountId === 'account-1' && blockPrimaryRefresh) return primaryDialogs.promise
			return Promise.resolve({
				item: [{
					providerChatId: `chat-${accountId}`,
					title: accountId === 'account-1' ? 'Primary chat' : 'Secondary chat',
					observedAtUnixSeconds: 1n,
				}],
			} as never)
		})
		vi.mocked(listWhatsAppOperationalMessages).mockImplementation(({ accountId, providerChatId }) => (
			Promise.resolve({ item: [{
				providerChatId,
				providerMessageId: `message-${accountId}`,
				text: accountId === 'account-1' ? 'Primary message' : 'Secondary message',
			}] } as never)
		))
		const controller = useWhatsAppOperationalRead({
			canQuery: () => true,
			modules: () => [whatsAppModule('whatsapp.operational.query.v1', 'account-1'), whatsAppModule('whatsapp.operational.query.v1', 'account-2')],
		})
		await controller.reconcile()
		await controller.selectAccount('account-2')
		blockPrimaryRefresh = true

		const refresh = controller.selectAccount('account-1')

		expect(controller.model.value.selectedAccountId).toBe('account-1')
		expect(controller.model.value.dialogs[0]?.title).toBe('Primary chat')
		primaryDialogs.resolve({ item: [{
			providerChatId: 'chat-account-1',
			title: 'Primary chat refreshed',
			observedAtUnixSeconds: 2n,
		}] } as never)
		await refresh
		expect(controller.model.value.dialogs[0]?.title).toBe('Primary chat refreshed')
	})

	it('fails closed before transport without capability or effective account', async () => {
		const blocked = useWhatsAppOperationalRead({
			canQuery: () => false,
			modules: () => [whatsAppModule('whatsapp.operational.query.v1')],
		})
		await blocked.reconcile()
		expect(blocked.model.value.state).toBe('blocked')

		const noAccount = useWhatsAppOperationalRead({
			canQuery: () => true,
			modules: () => [],
		})
		await noAccount.reconcile()
		expect(noAccount.model.value.state).toBe('empty')
		expect(getWhatsAppOperationalRuntimeStatus).not.toHaveBeenCalled()
	})
})

function whatsAppModule(capabilityId: string, accountId = 'account-1') {
	return create(ClientModuleBootstrapV1Schema, {
		registrationId: `whatsapp-${accountId}`,
		moduleId: 'makosh-whatsapp-runtime',
		sectionsEnabled: true,
		capabilityIds: [capabilityId],
		settings: create(ClientModuleSettingsBootstrapV1Schema, {
			values: [create(ClientSettingValueEntryV1Schema, {
				settingId: 'whatsapp.account_id',
				value: create(ClientSettingValueV1Schema, {
					value: { case: 'stringValue', value: accountId },
				}),
			})],
		}),
	})
}

function deferred<T>() {
	let resolve!: (value: T) => void
	const promise = new Promise<T>((accept) => { resolve = accept })
	return { promise, resolve }
}

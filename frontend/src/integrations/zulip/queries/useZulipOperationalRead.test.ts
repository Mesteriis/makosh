import { create } from '@bufbuild/protobuf'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	getZulipOperationalAccountStatus,
	listZulipOperationalConversations,
	listZulipOperationalEvents,
	listZulipOperationalMessages,
	searchZulipOperationalMessages,
} from '../api/zulipOperationalReadGateway'
import { openZulipOperationalRealtime } from '../api/zulipOperationalRealtime'
import { useZulipOperationalRead } from './useZulipOperationalRead'

vi.mock('../api/zulipOperationalReadGateway', () => ({
	getZulipOperationalAccountStatus: vi.fn(),
	listZulipOperationalConversations: vi.fn(),
	listZulipOperationalEvents: vi.fn(),
	listZulipOperationalMessages: vi.fn(),
	searchZulipOperationalMessages: vi.fn(),
}))

vi.mock('../api/zulipOperationalRealtime', () => ({
	openZulipOperationalRealtime: vi.fn(() => ({ close: vi.fn() })),
}))

describe('Zulip operational read controller', () => {
	beforeEach(() => {
		vi.clearAllMocks()
		vi.mocked(openZulipOperationalRealtime).mockReturnValue({ close: vi.fn() })
		vi.mocked(getZulipOperationalAccountStatus).mockResolvedValue({
			accountId: 'account-1',
			projectionReady: true,
			latestEventSequence: 4n,
			bindingRevision: 1n,
		} as never)
		vi.mocked(listZulipOperationalConversations).mockResolvedValue({
			item: [{
				providerConversationId: 'stream:1:topic',
				streamName: 'Engineering',
				topic: 'Clean room',
				latestEventSequence: 4n,
			}],
			nextCursor: 'conversations-2',
		} as never)
		vi.mocked(listZulipOperationalEvents).mockResolvedValue({ item: [] } as never)
		vi.mocked(listZulipOperationalMessages).mockResolvedValue({
			item: [{
				providerConversationId: 'stream:1:topic',
				providerMessageId: 'message-1',
				senderId: 'owner@example.com',
				content: 'Hello',
				attachment: [],
				reaction: [],
				lastEventSequence: 4n,
			}],
		} as never)
		vi.mocked(searchZulipOperationalMessages).mockResolvedValue({ item: [] } as never)
	})

	it('refreshes from shared realtime without clearing the visible conversation', async () => {
		let onProjectionChanged: ((revision: bigint) => void) | undefined
		vi.mocked(openZulipOperationalRealtime).mockImplementation((_accountId, input) => {
			onProjectionChanged = input.onProjectionChanged
			return { close: vi.fn() }
		})
		const controller = useZulipOperationalRead({
			canQuery: () => true,
			canRealtime: () => true,
			modules: () => [zulipModule('zulip.operational.query.v1')],
		})
		await controller.reconcile()
		vi.mocked(listZulipOperationalMessages).mockResolvedValue({
			item: [{
				providerConversationId: 'stream:1:topic',
				providerMessageId: 'message-2',
				senderId: 'owner@example.com',
				content: 'Updated',
				attachment: [],
				reaction: [],
				lastEventSequence: 5n,
			}],
		} as never)

		onProjectionChanged?.(2n)
		expect(controller.model.value).toMatchObject({
			state: 'ready',
			selectedConversationId: 'stream:1:topic',
		})
		await vi.waitFor(() => {
			expect(controller.model.value.messages[0]?.content).toBe('Updated')
		})
		controller.stopRealtime()
	})

	it('loads account, conversations, messages and search through exact gateways', async () => {
		const controller = useZulipOperationalRead({
			canQuery: () => true,
			modules: () => [zulipModule('zulip.operational.query.v1')],
		})

		await controller.reconcile()
		controller.updateSearchQuery(' Hello ')
		await controller.search()

		expect(getZulipOperationalAccountStatus).toHaveBeenCalledWith('account-1')
		expect(listZulipOperationalConversations).toHaveBeenCalledWith({
			accountId: 'account-1',
		})
		expect(listZulipOperationalMessages).toHaveBeenCalledWith({
			accountId: 'account-1',
			providerConversationId: 'stream:1:topic',
		})
		expect(searchZulipOperationalMessages).toHaveBeenCalledWith({
			accountId: 'account-1',
			providerConversationId: 'stream:1:topic',
			searchQuery: 'Hello',
		})
		expect(controller.model.value).toMatchObject({
			state: 'ready',
			selectedAccountId: 'account-1',
			selectedConversationId: 'stream:1:topic',
			hasMoreConversations: true,
		})
		expect(controller.model.value.messages[0]?.content).toBe('Hello')
	})

	it('loads the next page without duplicating an existing conversation', async () => {
		const controller = useZulipOperationalRead({
			canQuery: () => true,
			modules: () => [zulipModule('zulip.operational.query.v1')],
		})
		await controller.reconcile()
		vi.mocked(listZulipOperationalConversations).mockResolvedValueOnce({
			item: [{
				providerConversationId: 'stream:1:topic',
				latestEventSequence: 4n,
			}, {
				providerConversationId: 'direct:owner',
				latestEventSequence: 5n,
			}],
		} as never)

		await controller.loadMoreConversations()

		expect(listZulipOperationalConversations).toHaveBeenLastCalledWith({
			accountId: 'account-1',
			cursor: 'conversations-2',
		})
		expect(controller.model.value.conversations.map(({ id }) => id))
			.toEqual(['stream:1:topic', 'direct:owner'])
	})

	it('keeps the cached account projection visible while a returning account refreshes', async () => {
		const primaryConversations = deferred<never>()
		let blockPrimaryRefresh = false
		let refreshedPrimary = false
		vi.mocked(listZulipOperationalConversations).mockImplementation(({ accountId }) => {
			if (accountId === 'account-1' && blockPrimaryRefresh) return primaryConversations.promise
			return Promise.resolve({ item: [{
				providerConversationId: `conversation-${accountId}`,
				streamName: accountId === 'account-1' ? 'Primary stream' : 'Secondary stream',
				latestEventSequence: 1n,
			}] } as never)
		})
		vi.mocked(listZulipOperationalMessages).mockImplementation(({ accountId, providerConversationId }) => (
			Promise.resolve({ item: [{
				providerConversationId,
				providerMessageId: `message-${accountId}`,
				content: accountId === 'account-1'
					? (refreshedPrimary ? 'Primary message refreshed' : 'Primary message')
					: 'Secondary message',
				attachment: [],
				reaction: [],
			}] } as never)
		))
		const controller = useZulipOperationalRead({
			canQuery: () => true,
			modules: () => [zulipModule('zulip.operational.query.v1', 'account-1'), zulipModule('zulip.operational.query.v1', 'account-2')],
		})
		await controller.reconcile()
		await controller.selectAccount('account-2')
		blockPrimaryRefresh = true

		const refresh = controller.selectAccount('account-1')

		expect(controller.model.value.selectedAccountId).toBe('account-1')
		expect(controller.model.value.messages[0]?.content).toBe('Primary message')
		refreshedPrimary = true
		primaryConversations.resolve({ item: [{
			providerConversationId: 'conversation-account-1',
			streamName: 'Primary stream refreshed',
			latestEventSequence: 2n,
		}] } as never)
		await refresh
		expect(controller.model.value.messages[0]?.content).toBe('Primary message refreshed')
	})

	it('fails closed before transport without capability or effective account', async () => {
		const blocked = useZulipOperationalRead({
			canQuery: () => false,
			modules: () => [zulipModule('zulip.operational.query.v1')],
		})
		await blocked.reconcile()
		expect(blocked.model.value.state).toBe('blocked')

		const noAccount = useZulipOperationalRead({
			canQuery: () => true,
			modules: () => [],
		})
		await noAccount.reconcile()
		expect(noAccount.model.value.state).toBe('empty')
		expect(getZulipOperationalAccountStatus).not.toHaveBeenCalled()
	})
})

function zulipModule(capabilityId: string, accountId = 'account-1') {
	return create(ClientModuleBootstrapV1Schema, {
		registrationId: `zulip-${accountId}`,
		moduleId: 'makosh-zulip-runtime',
		sectionsEnabled: true,
		capabilityIds: [capabilityId],
		settings: create(ClientModuleSettingsBootstrapV1Schema, {
			values: [create(ClientSettingValueEntryV1Schema, {
				settingId: 'zulip.account_id',
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

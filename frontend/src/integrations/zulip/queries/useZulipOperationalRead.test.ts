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
import { useZulipOperationalRead } from './useZulipOperationalRead'

vi.mock('../api/zulipOperationalReadGateway', () => ({
	getZulipOperationalAccountStatus: vi.fn(),
	listZulipOperationalConversations: vi.fn(),
	listZulipOperationalEvents: vi.fn(),
	listZulipOperationalMessages: vi.fn(),
	searchZulipOperationalMessages: vi.fn(),
}))

describe('Zulip operational read controller', () => {
	beforeEach(() => {
		vi.clearAllMocks()
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

function zulipModule(capabilityId: string) {
	return create(ClientModuleBootstrapV1Schema, {
		registrationId: 'zulip-primary',
		moduleId: 'makosh-zulip-runtime',
		sectionsEnabled: true,
		capabilityIds: [capabilityId],
		settings: create(ClientModuleSettingsBootstrapV1Schema, {
			values: [create(ClientSettingValueEntryV1Schema, {
				settingId: 'zulip.account_id',
				value: create(ClientSettingValueV1Schema, {
					value: { case: 'stringValue', value: 'account-1' },
				}),
			})],
		}),
	})
}

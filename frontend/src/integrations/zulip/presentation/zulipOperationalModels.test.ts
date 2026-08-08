import { describe, expect, it } from 'vitest'

import { ZulipCredentialBindingStateV1 } from '../../../gen/makosh/zulip/account/v1/client_pb'
import {
	ZulipConversationKindV1,
	ZulipHistoryStateV1,
	ZulipOperationalEventKindV1,
} from '../../../gen/makosh/zulip/operational/v1/client_pb'
import { buildZulipOperationalReadModel } from './zulipOperationalReadModel'
import { buildZulipOperationalReplayModel } from './zulipOperationalReplayModel'

describe('Zulip operational presentation models', () => {
	it('maps account/history and provider-owned message records', () => {
		const model = buildZulipOperationalReadModel({
			canQuery: true,
			state: 'ready',
			statusMessage: '',
			accounts: [{ accountId: 'account-1', registrationId: 'registration-1' }],
			selectedAccountId: 'account-1',
			selectedConversationId: 'stream:1:topic',
			searchQuery: '',
			accountStatus: {
				projectionReady: true,
				historyState: ZulipHistoryStateV1.ZULIP_HISTORY_STATE_READY,
				credentialState:
					ZulipCredentialBindingStateV1.ZULIP_CREDENTIAL_BINDING_STATE_ACTIVE,
				latestEventSequence: 7n,
				bindingRevision: 2n,
			} as never,
			conversations: [{
				providerConversationId: 'stream:1:topic',
				kind: ZulipConversationKindV1.ZULIP_CONVERSATION_KIND_STREAM_TOPIC,
				streamName: 'Engineering',
				topic: 'Clean room',
				latestEventSequence: 7n,
			} as never],
			messages: [{
				providerMessageId: 'message-1',
				providerConversationId: 'stream:1:topic',
				senderId: 'owner@example.com',
				content: '**Hello**',
				sentAtUnixSeconds: 1_700_000_000n,
				attachment: [],
				reaction: [],
				lastEventSequence: 7n,
			} as never],
			events: [{
				providerEventId: 9n,
				providerMessageId: 'message-1',
				kind: ZulipOperationalEventKindV1.ZULIP_OPERATIONAL_EVENT_KIND_MESSAGE_UPDATED,
				content: 'Updated',
				observedAtUnixSeconds: 1_700_000_000n,
			} as never],
			searchResults: [],
			hasMoreConversations: false,
			hasMoreMessages: false,
			hasMoreEvents: false,
			hasMoreSearchResults: false,
		})

		expect(model.accountStatus).toMatchObject({
			projectionState: 'Ready',
			historyState: 'Ready',
			credentialState: 'Active',
		})
		expect(model.conversations[0]).toMatchObject({
			title: 'Engineering / Clean room',
			selected: true,
		})
		expect(model.messages[0]).toMatchObject({
			sender: 'owner@example.com',
			content: '**Hello**',
		})
		expect(model.events[0]).toMatchObject({
			kind: 'Message updated',
			summary: 'Updated',
		})
	})

	it('preserves explicit replay reset and cursor semantics', () => {
		const model = buildZulipOperationalReplayModel({
			canReplay: true,
			state: 'error',
			statusMessage: 'reset',
			accounts: [{ accountId: 'account-1', registrationId: 'registration-1' }],
			selectedAccountId: 'account-1',
			earliestSequence: 5n,
			latestSequence: 8n,
			nextSequence: 0n,
			resetRequired: true,
			frames: [],
		})
		expect(model).toMatchObject({
			earliestSequence: '5',
			latestSequence: '8',
			nextSequence: '0',
			resetRequired: true,
			hasMore: false,
		})
	})
})

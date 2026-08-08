import { create, toBinary } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'

import {
	MailContactsSyncErrorCodeV1,
	MailContactsSyncStateV1,
	MailContactsSyncStatusChangedV1Schema,
} from '../../../gen/makosh/mail_contacts_sync/v1/sync_pb'
import {
	ClientRealtimeEventV1Schema,
	ClientRealtimeStreamStateKindV1,
	ClientRealtimeStreamStateV1Schema,
} from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import type { BrowserGatewayRealtimeObserver } from '../../../platform/gateway/browserGatewayRealtime'
import { openMailContactsSyncRealtime } from './mailContactsSync'

describe('Mail Contacts Sync realtime adapter', () => {
	it('opens shared realtime before Start and replays a buffered exact-run status', async () => {
		let sourceObserver: BrowserGatewayRealtimeObserver | undefined
		const close = vi.fn()
		const hub = {
			subscribe: vi.fn((observer: BrowserGatewayRealtimeObserver) => {
				sourceObserver = observer
				return { close }
			}),
		}
		const observer = { onStatus: vi.fn(), onUnavailable: vi.fn() }
		const binding = openMailContactsSyncRealtime(observer, hub)
		sourceObserver?.onStreamState(create(ClientRealtimeStreamStateV1Schema, {
			state: ClientRealtimeStreamStateKindV1.CLIENT_REALTIME_STREAM_STATE_KIND_OPEN,
		}))
		await expect(binding.ready).resolves.toBeUndefined()

		const runId = new Uint8Array(16).fill(7)
		const status = create(MailContactsSyncStatusChangedV1Schema, {
			runId,
			state: MailContactsSyncStateV1.MAIL_CONTACTS_SYNC_STATE_ACCEPTED,
			stateRevision: 1n,
			error: MailContactsSyncErrorCodeV1.MAIL_CONTACTS_SYNC_ERROR_CODE_UNSPECIFIED,
		})
		sourceObserver?.onEvent(create(ClientRealtimeEventV1Schema, {
			contractName: 'mail_contacts_sync_realtime',
			contractVersion: 1,
			eventKind: 'mail.contacts-sync.status-changed.v1',
			payload: toBinary(MailContactsSyncStatusChangedV1Schema, status),
		}))
		expect(observer.onStatus).not.toHaveBeenCalled()
		binding.attachRun(runId)
		expect(observer.onStatus).toHaveBeenCalledWith(status)
		binding.close()
		expect(close).toHaveBeenCalledTimes(1)
	})
})
